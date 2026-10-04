#!/usr/bin/env bash

{
    # request with body, then request without body, all at once
    printf 'GET http://httpbin.org/get?request=one HTTP/1.1\r\nHost: httpbin.org\r\nContent-Length: 5\r\n\r\nhelloGET http://httpbin.org/get?request=two HTTP/1.1\r\nHost: httpbin.org\r\n\r\n'
    #
    # 2 requests without body, all at once
    # printf 'GET http://httpbin.org/get?request=one HTTP/1.1\r\nHost: httpbin.org\r\n\r\nGET http://httpbin.org/get?request=two HTTP/1.1\r\nHost: httpbin.org\r\n\r\n'
    #
    # 2 requests, with a gap in the middle of the first request
    # printf 'GET http://httpbin.org/get?request=one HTTP/1.1\r\nHost: httpbin.org\r\n'
    # sleep 1
    # printf '\r\nGET http://httpbin.org/get?request=two HTTP/1.1\r\nHost: httpbin.org\r\n\r\n'
    #
    # 2 requests, with a gap in the middle of the second request
    # printf 'GET http://httpbin.org/get?request=one HTTP/1.1\r\nHost: httpbin.org\r\n\r\nGET h'
    # sleep 1
    # printf 'ttp://httpbin.org/get?request=two HTTP/1.1\r\nHost: httpbin.org\r\n\r\n'
    #
    # request with body, then request without body
    # printf 'GET http://httpbin.org/get?request=one HTTP/1.1\r'
    # printf '\nHost: httpbin.org\r\n'
    # printf '\nContent-Length: 5\r\n'
    # printf '\r\n'
    # printf 'hello'
    # printf 'GET http://httpbin.org/get?request=two HTTP/1.1\r\n'
    # printf 'Host: httpbin.org\r\n'
    # printf '\r\n'
    #
    # single request with body split
    # printf 'GET http://httpbin.org/get?request=one HTTP/1.1\r\n'
    # printf 'Host: httpbin.org\r\n'
    # printf 'Content-Length: 5\r\n'
    # printf '\r\n'
    # printf 'he'
    # sleep 1
    # printf 'llo'
} | nc 127.0.0.1 8080
