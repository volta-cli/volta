FROM alpine
RUN apk update && apk add bash openssl curl

#FROM ubuntu
#FROM debian
#RUN apt-get update && apt-get install -y openssl curl

COPY ./dev/unix/volta-install.sh /
RUN cat /volta-install.sh | bash
