-- Generated by: Claude3
-- 我正在使用PostgreSQL 14，请你为我编写一段sql代码：
-- 1：创建数据库user，其中包含自增的主键id，一个字符串类型储存用户的密钥，用户当前是否可用
-- 2：创建用户使用记录user_usage，其中包含外键用户id，两个数字类型字段用于储存用户已经使用的输入、输出的token总用量。此外还有用户购买的总用量，使用数字类型储存
-- 3：创建用户日志表usage_list，其中包含外键用户id，当前记录的自增主键，记录的时间，以及输入、输出的token用量，以及输入、输出token的单价是多少
-- 4：对话id记录表chat_list，其中包含自增的主键id，外键绑定的账户id，外键id，ai_output，user_input两个字符串字段，记录的时间
-- 5：可用账户列表account_list，其中包含自增的主键id，是否被禁用，用户名，密码，账户类型字段"Endpoint"
-- 其中，当表三增加记录时，表二对应用户的已使用次数要自动增加，同时通过本次记录使用的输入、输出token和它们对应的单价，在usage中进行扣费。
-- 当user表添加或删除用户时，user_usage应该自动增加或删除记录，当user_usage中的money字段小于等于0时user变为不可用状态。


-- 创建用户表
CREATE TABLE "user" (
                        id SERIAL PRIMARY KEY,
                        api_key VARCHAR(255) NOT NULL,
                        is_active BOOLEAN NOT NULL DEFAULT TRUE
);

-- 创建用户使用记录表
CREATE TABLE user_usage (
                            usage_id SERIAL PRIMARY KEY,
                            user_id INTEGER REFERENCES "user"(id),
                            total_input_tokens BIGINT DEFAULT 0 NOT NULL,
                            total_output_tokens BIGINT DEFAULT 0 NOT NULL,
                            total_purchased NUMERIC DEFAULT 10 NOT NULL
);

-- 创建用户日志表
CREATE TABLE usage_list (
                            id SERIAL PRIMARY KEY,
                            user_id INTEGER REFERENCES "user"(id),
                            timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                            input_tokens INTEGER NOT NULL,
                            output_tokens INTEGER NOT NULL,
                            input_token_price NUMERIC NOT NULL,
                            output_token_price NUMERIC NOT NULL
);

-- 创建可用账户列表
CREATE TABLE account_list (
                              id SERIAL PRIMARY KEY,
                              is_disabled BOOLEAN NOT NULL DEFAULT FALSE,
                              use_proxy VARCHAR(255),
                              api_key VARCHAR(255) NOT NULL,
                              endpoint VARCHAR(255) NOT NULL
);

-- 创建对话id记录表
CREATE TABLE chat_list (
                           id SERIAL PRIMARY KEY,
                           account_id INTEGER REFERENCES account_list(id),
                           user_id INTEGER REFERENCES "user"(id),
                           message_key VARCHAR(255) NOT NULL,
                           ai_output VARCHAR(255) NOT NULL,
                           user_input VARCHAR(255) NOT NULL,
                           timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE public_command (
                        id SERIAL PRIMARY KEY,
                        command varchar(50) NOT NULL,
                        describe TEXT NOT NULL,
                        prompt TEXT NOT NULL,
                        is_disable BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE private_command (
                                id SERIAL PRIMARY KEY,
                                user_id INTEGER REFERENCES "user"(id),
                                command varchar(50) NOT NULL,
                                describe TEXT NOT NULL,
                                prompt TEXT NOT NULL,
                                is_disable BOOLEAN NOT NULL DEFAULT FALSE
);

-- 创建触发器函数，在向 usage_list 表插入记录时自动更新 user_usage 表
CREATE FUNCTION update_user_usage() RETURNS TRIGGER AS $$
BEGIN
UPDATE user_usage
SET total_input_tokens = total_input_tokens + NEW.input_tokens,
    total_output_tokens = total_output_tokens + NEW.output_tokens,
    total_purchased = total_purchased - (NEW.input_tokens * NEW.input_token_price + NEW.output_tokens * NEW.output_token_price)
WHERE user_id = NEW.user_id;

RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 创建触发器，在向 usage_list 表插入记录时触发 update_user_usage 函数
CREATE TRIGGER update_user_usage_trigger
    AFTER INSERT ON usage_list
    FOR EACH ROW
    EXECUTE FUNCTION update_user_usage();

-- 创建触发器函数，在向 "user" 表插入记录时自动在 user_usage 表中添加对应记录
CREATE FUNCTION add_user_usage() RETURNS TRIGGER AS $$
BEGIN
INSERT INTO user_usage (user_id) VALUES (NEW.id);
RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 创建触发器，在向 "user" 表插入记录时触发 add_user_usage 函数
CREATE TRIGGER add_user_usage_trigger
    AFTER INSERT ON "user"
    FOR EACH ROW
    EXECUTE FUNCTION add_user_usage();

-- 创建触发器函数，在从 "user" 表删除记录时自动从 user_usage 表中删除对应记录
CREATE FUNCTION delete_user_usage() RETURNS TRIGGER AS $$
BEGIN
DELETE FROM user_usage WHERE user_id = OLD.id;
RETURN OLD;
END;
$$ LANGUAGE plpgsql;

-- 创建触发器，在从 "user" 表删除记录时触发 delete_user_usage 函数
CREATE TRIGGER delete_user_usage_trigger
    AFTER DELETE ON "user"
    FOR EACH ROW
    EXECUTE FUNCTION delete_user_usage();

-- 创建触发器函数，当 user_usage 表中的 total_purchased 字段小于等于 0 时将对应用户的 is_active 字段设置为 false
CREATE FUNCTION deactivate_user() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.total_purchased <= 0 THEN
UPDATE "user" SET is_active = false WHERE id = NEW.user_id;
END IF;
RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 创建触发器，在更新 user_usage 表时触发 deactivate_user 函数
CREATE TRIGGER deactivate_user_trigger
    AFTER UPDATE ON user_usage
    FOR EACH ROW
    EXECUTE FUNCTION deactivate_user();
