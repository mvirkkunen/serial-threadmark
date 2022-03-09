serial-threadmark
=================

Simple program for testing serial loopback devices for correctness and weeding out race conditions.

Usage example:

```  
> serial-threadmark /dev/ttyUSB0 9600 1000000
# or 
> cargo run --release -- /dev/ttyUSB0 9600 1000000
```