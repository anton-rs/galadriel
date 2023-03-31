# `op-challenger-driver`

The `op-challenger-driver` crate contains the [Driver] trait and its various implementations. A [Driver]'s main role is to maintain an
async state loop that triggers actions upon receiving events.

When the `op-challenger` binary is ran, it will create [Driver] instances based on the configuration and run the driver loops in parallel.
