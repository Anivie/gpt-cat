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