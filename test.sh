#!/usr/bin/env bash

{
    printf 'GET http://httpbin.org/get?request=one HTTP/1.1\r'
    sleep 1
    printf '\nHost: httpbin.org\r\n'
    printf '\nContent-Length: 5\r\n'
    sleep 1
    printf '\r\n'
    printf 'hello'
    sleep 1
    printf 'GET http://httpbin.org/get?request=one HTTP/1.1\r\n'
    printf 'Host: httpbin.org\r\n'
    printf '\r\n'
} | nc 127.0.0.1 8080
