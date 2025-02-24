This is a small example of using the new `nabob` dependency. This shall be removed once we have
documentation/tests.

`pack2` contains a package which is used by `pack1` as follows:

```
[dependencies]
Pack2 = { nabob = "http://localhost:8080", address = "default" }
```

To see it working:

```shell
# Start a node with an account
nabob node run-local-testnet &
nabob account create --account default --use-faucet 
# Compile and publish pack2
cd pack2
nabob move compile --named-addresses project=default     
nabob move publish --named-addresses project=default
# Compile pack1 agains the published pack2
cd ../pack1
nabob move compile --named-addresses project=default     
```
