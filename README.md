# mikage

Spotifyでログイン(mikageサインアップ) -> Twitterでログイン(紐付け) -> 一定間隔でタイムラインを取得し続けて楽曲を収集する

- rustc 1.65.0
- sea-orm-cli 0.10.3

## directory

- api ... axumでapiを書く
- core ... db操作とかspotifyのプレイリスト操作を行う
- entity ... dbのモデルを書く?
- ./src ... 全部巻き込んでサーバーを立てたりする

## db

- sqlx
- postgres

誰か
User [ user_id, username, created_at, updated_at, activated_at ]

誰のSpotifyアカウントか
SpotifyAccount [ spotify_id, user_id, access_token, refresh_token, created_at, updated_at ]

誰のTwitterアカウントか
TwitterAccount [ twitter_id, user_id, access_token, refresh_token, created_at, updated_at ]

誰が拾った楽曲か、どこがソースか
Track [ id, user_id, track_url, source_url, created_at ]

## routes

- actix_webで行く

- /login -> redirect spotify
- /callback -> get spotify code
- /twitter/login -> redirect twitter
- /twitter/callback -> get twitter code

## task

わからん
