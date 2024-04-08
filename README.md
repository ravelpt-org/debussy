# Debussy

Debussy is the judging software for [Ravel](https://github.com/TimberCreekProgrammingTeam/ravel), written in rust.
Debussy uses Docker to isolate the environment for each submission.
Debussy uses [Reverie](https://github.com/TimberCreekProgrammingTeam/reverie) as the docker container.

## Getting Started

##### (More streamlined installation options coming soon!)

In order to get started with Debussy, you first need [Docker](https://docs.docker.com/get-docker/), along with [Rust](https://www.rust-lang.org/tools/install).
You need to do some configuration with docker so that Debussy can communicate with it.

```vi /lib/systemd/system/docker.service``` (Replace vi with your editor of choice)

Add ```-H=tcp://0.0.0.0:2375``` to the ExecStart line and then save.
Now run 

```
systemctl daemon-reload
sudo service docker restart
sudo service docker restart
```
(You can test this by continuing with this guide or by running the following command) 

```curl http://localhost:2375/images/json```

After installing Docker and Rust, clone this repo. Once cloned create a .env at the root of the repository, which mimics this sample (the max_value in the example is arbitrary).
```
ravel_url=http://0.0.0.0:8000
ravel_username=judge
ravel_password=*****
max_jobs=64
```
Once the .env is setup you just need to run
```cargo run --release```