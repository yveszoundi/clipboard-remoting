openssl req -new -subj "/C=CA/CN=localhost" -newkey rsa:4096 -nodes -x509 -days 60  -keyout pem-cert-priv.pem -out pem-cert-pub.pem -addext "subjectAltName=DNS:localhost,IP:127.0.0.1,IP:10.0.2.2"
openssl x509 -outform der -in der-cert-pub.pem -out der-cert-pub.der
openssl rsa -inform pem -in  der-cert-priv.pem -outform der -out der-cert-priv.der
