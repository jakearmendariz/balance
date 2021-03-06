# Loadbalancer for a distributed key value store

Based upon distributed kvs I built in python. This binary will forward requests and using consistent hashing distribute the amount of keys per shard. Essentially I wanted an endpoint that could do the work of consistent hashing faster than my python program, and I can use to encrypt information sending to the store.
