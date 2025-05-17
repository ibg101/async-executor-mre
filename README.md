## This repo was initally created as MRE.

https://stackoverflow.com/questions/79625982/async-executor-terminates-early-despite-waker-clone-in-future

However i decided to [explain](https://github.com/ibg101/async-executor-mre/blob/main/src/runtime/core.rs#L59) it by myself why the UB occurs and how it can be fixed in 3 ways.
