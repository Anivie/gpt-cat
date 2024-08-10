FROM ubuntu:latest

# 创建一个工作目录
WORKDIR /app

# 复制应用代码到工作目录
COPY ./target/release/gpt-cat /app
COPY ./config/config.json /app/config/
COPY ./config/model_price.json /app/config/
COPY ./config/model.json /app/config/

RUN chmod 777 ./gpt-cat

CMD ["./gpt-cat"]