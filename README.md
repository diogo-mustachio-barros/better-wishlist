# Commands

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

## List wishlist `.wl`

# How to Run
BetterWishlist requires two credentials:
- Discord bot token
- MongoDB URI 

These secrets can be passed directy through the terminal in the presented order, or added as
    environment variables with names `DISCORD_TOKEN` and `MONGODB_URL` respectively.