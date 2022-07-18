# mikage

タイムラインに流れている Spotify とかの URL を集めて、 Spotify のプレイリストに突っ込むやつ

## 使い方

1. ここのリポジトリをクローンして、ビルドする
2. 設定ファイルを追加する -> [設定ファイルについて](#設定ファイルについて)
3. 引数に設定ファイルのパスを指定して実行

## やっとること

- もし、設定ファイルに Spotify の `access_token` がない場合は、認証用の URL を CLI に出力するので、それをブラウザで開いて認証し、リダイレクトされた URL をコピってそのまま CLI に入力する
- `refresh_token` まである場合は実行時に勝手にリフレッシュする
- とってきたトークン類はそのまま設定ファイルに書き込まれる
- Twitter に対しては何もしない
  * [ekuinox/tomoe](https://github.com/ekuinox/tomoe)が書き込んだ JSON から Twitter の `access_token` をのぞき見する
  * なので tomoe が要ります

## 設定ファイルについて

JSON 形式で記述する

- `spotify_playlist_id` には、トラックを追加する対象のプレイリストを指定する（なのであらかじめ作っといて欲しい
- `log_file` はログの出力先 指定しなくてもいい

```json
{
  "credentials": [
    [
      "twitter",
      "../tomoe/secrets.json"
    ],
    {
      "service": "spotify",
      "client_id": "<SPOTIFY_CLIENT_ID>",
      "client_secret": "<SPOTIFY_CLIENT_SECRET>",
      "callback_url": "<SPOTIFY_CALLBACK_URL>",
    }
  ],
  "spotify_playlist_id": "<PLAYLIST_ID>",
  "log_file": "./mikage.log"
}
```
