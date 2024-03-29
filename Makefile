# musl / gnu
# Use musl for best compatability, since it's static-linked.
platform=musl

# function-name = merkle_server
# function-role = arn:aws:iam::xxxxxx
-include lambda.mk

# Use cross to cross-build
# https://github.com/cross-rs/cross
lambda-build:
	cross build --target x86_64-unknown-linux-${platform} --release

lambda-pack: lambda-build
	zip -j target/lambda.zip ./target/x86_64-unknown-linux-${platform}/release/bootstrap

lambda-create: lambda-pack
	aws lambda create-function --function-name ${function-name} \
	--handler lambda \
	--zip-file fileb://./target/lambda.zip \
	--runtime provided.al2 \
	--role ${function-role} \
	--environment Variables={RUST_BACKTRACE=1} \
	--tracing-config Mode=Active

lambda-update: lambda-pack
	@aws lambda update-function-code --function-name ${function-name} --zip-file 'fileb://./target/lambda.zip'

lambda-delete:
	@aws lambda delete-function --function-name ${func}

lambda-container-build:
	@podman build --platform linux/amd64 --build-arg=COMMIT=$(git rev-parse --short HEAD) --build-arg=NOW=$(date +%s) -t nextid/kv-server-lambda:latest .

pg-connect:
	@echo "kv_server\n" | docker-compose exec pg psql -Ukv_server kv_server_development

test:
	@RUST_BACKTRACE=1 RUST_LOG=debug cargo test -- --nocapture
