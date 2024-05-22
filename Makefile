launch: clean
	docker network rm --force ztclient-tester
	docker network create ztclient-tester
	docker compose up --detach

create-env: launch
	cargo run -- create-environment

test: create-env
	cargo test

nextest: create-env
	cargo nextest run

clean:
	docker ps -aq  | xargs docker rm -f
	rm -rf running.json

start-new-container:
	docker container run --detach --network ztclient-tester ztclient-nginx:latest --tun userspace-networking --statedir /run/ztclientd

kill-clients:
	docker ps | grep ztclient-nginx | awk '{ print $1 }' | xargs docker rm -f

pull-images:
	docker compose pull --ignore-pull-failures
