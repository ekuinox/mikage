# mikage

Spotifyでログイン(mikageサインアップ) -> Twitterでログイン(紐付け) -> 一定間隔でタイムラインを取得し続けて楽曲を収集する

## db

- sqlx
- postgres ?

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

- /api/**/*
- /api/callback/twitter
- /api/callback/spotify
- /login -> redirect spotify
- /connect/twitter -> redirect twitter
- /* -> staticfile

## task

わからん

## front

- vite + react
- mui ?
- swr ?

これはまあなんでもいい
