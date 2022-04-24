#!/usr/bin/env sh
SSL_COMMAND="openssl"

if [ `uname` = "Darwin" ]
then
    # adjust the path on MacOS to point to a recent openssl version
    SSL_COMMAND="${HOME}/Tools/homebrew/Cellar/openssl@1.1/1.1.1n/bin/openssl"
fi

${SSL_COMMAND} req -new -subj "/C=CA/CN=localhost" -newkey rsa:4096 -nodes -x509 -days 60  -keyout key.pem -out cert.pem -addext "subjectAltName=DNS:localhost,IP:127.0.0.1,IP:10.0.2.2"
${SSL_COMMAND} x509 -outform der -in der-cert-pub.pem -out der-cert-pub.der
${SSL_COMMAND} rsa -inform pem -in  der-cert-priv.pem -outform der -out der-cert-priv.der
