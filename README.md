# Submission for GameOff 2022

Name and game are work in progress ;)

Currently broken in Firefox, but works in single and multiplayer mode in Chrome. Started from [extreme_bevy](https://github.com/johanhelsing/extreme_bevy).

## Local dev in the browser

`cargo watch -cx "run --release --target wasm32-unknown-unknown --config 'target.wasm32-unknown-unknown.runner = \"wasm-server-runner\"'"`

## Deployed dev build (might be outdated)

https://niklasei.github.io/bevy_boxhead/


## Nginx setup for matchbox server

 Because it took me way too long to get this running:
 ```
upstream appserver {
    server 127.0.0.1:3536;
}

server {
  listen 443 ssl;
  server_name match.nikl.me;

  location / {
    proxy_pass http://appserver;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "Upgrade";
    proxy_set_header Host $host;
  }
    ssl_certificate /etc/letsencrypt/live/match.nikl.me/fullchain.pem; # managed by Certbot
    ssl_certificate_key /etc/letsencrypt/live/match.nikl.me/privkey.pem; # managed by Certbot
    include /etc/letsencrypt/options-ssl-nginx.conf; # managed by Certbot
    ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem; # managed by Certbot
}
 ```
