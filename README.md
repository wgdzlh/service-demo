# 项目说明

本项目为Rust后端Web服务DEMO

## 准备本地测试用Postgres数据库实例（可选）
```sh
docker run -d --restart always --network host --name postgres-test -e TZ=Asia/Shanghai -e PGTZ=Asia/Shanghai -e POSTGRES_PASSWORD=mysecretpassword -e PGDATA=/var/lib/postgresql/data/pgdata -v /home/wgdzlh/docker/postgres:/var/lib/postgresql/data postgres:11-bullseye
```

## 本地调试
```sh
sudo sed -i '$ a 127.0.0.1 host.docker.internal' /etc/hosts  # 或修改config.toml
cargo run -- -l
```

## 本地使用docker容器调试
```sh
make up
```

## 发布docker镜像
```sh
make release
```
