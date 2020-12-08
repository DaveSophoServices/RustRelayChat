-- initialize the chat
CREATE DATABASE chat;

-- select it for use
USE chat;

CREATE TABLE chat_log (
       id    	bigint NOT NULL AUTO_INCREMENT PRIMARY KEY,
       stamp    timestamp,
       username varchar(100),
       address  varchar(30),
       channel  varchar(50),
       message  varchar(1024)
       );
