# geoip-loki

* performs city lookup of IP
* reports results to loki

```console
$ cargo run -- --loki-url http://loki.k3s.home/loki/api/v1/push --database ../GeoLite2-City.mmdb 73.15.180.224

    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
     Running `target/debug/geoip-loki --loki-url 'http://loki.k3s.home/loki/api/v1/push' --database ../GeoLite2-City.mmdb 73.15.180.224`
sent ok
```

* [GeoLite2 free geolocation data](https://dev.maxmind.com/geoip/geolite2-free-geolocation-data?lang=en)
* [loki push endpoint](https://grafana.com/docs/loki/latest/api/#push-log-entries-to-loki)
