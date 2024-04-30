#![allow(dead_code)]

use std::{collections::{HashMap, HashSet}, pin::Pin, sync::Arc, vec};

use serenity::{all::{Context, EventHandler, Http, Ready, UserId}, async_trait};
use serenity::model::channel::Message;
use serenity::prelude::*;

use super::logger::Logger;

type StatefulHandler<S> = for<'a> fn(&'a S, &'a Http, &'a Message) -> 
    Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>>;

pub struct MessageRouter<State, Log> 
    where Log: Logger + Send + Sync,
    State: Send + Sync
{
    logger: Arc<Log>,
    state: State,
    triggers: HashMap<String, Trigger<State>>
}

#[async_trait]
impl <State, Log> EventHandler for MessageRouter<State, Log>
    where Log: Logger + Send + Sync,
    State: Send + Sync
{
    async fn ready(&self, _:Context, _:Ready) {
        self.logger.log_info("Connected to Discord");
    }

    async fn message(&self, ctx: Context, msg: Message) {
        for (name, trigger) in self.triggers.iter() {
            if trigger.try_activate(&self.state, ctx.http(), &msg).await {
                self.logger.log_info(format!("activated {name}"));
                return;
            }
        }
    }
}


pub struct MessageRouterBuilder<State> {
    triggers: HashMap<String, Trigger<State>>,
    working_trigger: Option<Trigger<State>>
}

impl <State> MessageRouterBuilder<State>
    where State: Send + Sync
{
    pub fn new() -> Self {
        MessageRouterBuilder{
            triggers: HashMap::new(),
            working_trigger: None
        }
    }

    pub fn message<S: Into<String>>(self, id: S) -> Self {
        self.new_trigger(id, TriggerType::MESSAGE)
    }

    pub fn command<S: Into<String>>(self, id: S) -> Self {
        self.new_trigger(id, TriggerType::COMMAND)
    }

    pub fn from_user(mut self, user_id: UserId) -> Self {
        self.working_trigger = self.working_trigger.map(
            |trigger| trigger.add_user_id(user_id)
        );

        self
    }

    pub fn prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.working_trigger = self.working_trigger.map(
            |trigger| trigger.set_prefix(prefix)
        );

        self
    }

    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.working_trigger = self.working_trigger.map(
            |trigger| trigger.set_description(description)
        );

        self
    }

    pub fn no_bots(mut self) -> Self {
        self.working_trigger = self.working_trigger.map(
            |trigger| trigger.add_predicate(|msg: &Message| !msg.author.bot)
        );

        self
    }

    pub fn handler(mut self, handler: StatefulHandler<State>) -> Self {
        self.working_trigger = self.working_trigger.map(
            |trigger| trigger.set_handler(handler)
        );

        self
    }

    pub fn build<Log>(mut self, initial_state: State, logger: Arc<Log>) -> MessageRouter<State, Log> 
        where Log: Logger + Send + Sync
    {
        self = self.save_working_trigger();

        MessageRouter {
            state: initial_state,
            triggers: self.triggers,
            logger
        }
    }


    fn new_trigger<S: Into<String>>(mut self, id: S, trigger_type: TriggerType) -> Self {
        self = self.save_working_trigger();
        
        let new_id = id.into();
        if !self.triggers.contains_key(&new_id) {
            let trigger = Trigger::new(new_id, trigger_type);
            self.working_trigger = Some(trigger);
        }

        self
    }

    fn save_working_trigger(mut self) -> Self {
        if let Some(trigger) = self.working_trigger {
            self.triggers.insert(trigger.id.clone(), trigger);
            self.working_trigger = None
        }

        self
    }
}

struct Trigger<State>  {
    id: String,
    trigger_type: TriggerType,
    users: HashSet<UserId>,
    prefix: Option<String>,
    description: Option<String>,
    handler: Option<StatefulHandler<State>>,
    predicates: Vec<fn(&Message) -> bool>
}

enum TriggerType {
    MESSAGE,
    COMMAND
}

impl <State> Trigger<State> {
    fn new<S: Into<String>>(id: S, trigger_type: TriggerType) -> Self {
        Trigger { 
            id: id.into(),
            trigger_type,
            users: HashSet::new(),
            prefix: None,
            description: None,
            handler: None,
            predicates: vec![]
        }
    }

    fn add_user_id(mut self, user_id: UserId) -> Self {
        self.users.insert(user_id);
        
        self
    }

    fn set_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.prefix = Some(prefix.into());

        self
    }

    fn set_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());

        self
    }

    fn add_predicate(mut self, predicate: fn(&Message) -> bool) -> Self {
        self.predicates.push(predicate);
        
        self
    }

    fn set_handler(mut self, handler: StatefulHandler<State>) -> Self {
        self.handler = Some(handler);

        self
    }

    async fn try_activate(&self, state: &State, http: &Http, msg: &Message) -> bool {
        if
            // must have a handler to be activated
            self.handler.is_none()
            // author id must be in the users set
            || (!self.users.is_empty() && !self.users.contains(&msg.author.id))
            // message must contain the given prefix (if specified)
            || self.prefix.clone().is_some_and(|prefix| !msg.content.starts_with(&prefix))
            // check all predicates
            || !self.predicates.iter().all(|predicate| predicate(msg))
        {
            return false;
        }

        (self.handler.unwrap())(state, http, msg).await;

        return true;
    }
}