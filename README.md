# Commands

## Show help `.help`
Shows information on available commands.

## Add card to wishlist `.wa`
Single card:
```
.wa One Punch Man || Saitama
``` 

Multiple cards:
```
.wa One Piece || Monkey D. Luffy, Roronoa Zoro
```

## Remove from wishlist `.wr`
Single card:
```
.wr One Piece || Monkey D. Luffy
``` 

Multiple cards:
```
.wr One Piece || Monkey D. Luffy, Roronoa Zoro
```

A complete series:
```
.wr One Piece
```

## List wishlist `.wl`
List all wishlisted series:
```
.wl
```

List all wishlisted cards from a series:
```
.wl Mashle
```

List series from another user:
```
.wl @GokuEnjoyer Dragon Ball
```

# Reactions

## Drop pings
When a user is pinged on a drop, the respective card can be immediately removed from the wishlist
    by reacting with the respective ordinal emoji (1️⃣, 2️⃣, or 3️⃣).

## SOFI `ssl`
When you do a **series lookup** using SOFI, you can use reactions to:
- ✅ add unowned cards to your wishlist 
- ❌ remove owned cards from your wishlist

## SOFI `sg`
When trading cards, if the recipient has the card in its wishlist it can be removed by reacting 
    with ❌.

# How to Run
BetterWishlist requires two credentials:
- Discord bot token
- MongoDB URI 

These secrets can be passed directy through the terminal in the presented order, or added as
    environment variables with names `DISCORD_TOKEN` and `MONGODB_URL` respectively.

Run the bot with `cargo run`.