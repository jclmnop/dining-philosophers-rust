# The Dining Philosophers Problem

> Five philosophers dine together at the same table. Each philosopher has 
> their own place at the table. There is a fork between each plate. The dish 
> served is a kind of spaghetti which has to be eaten with two forks. Each 
> philosopher can only alternately think and eat. Moreover, a philosopher can 
> only eat their spaghetti when they have both a left and right fork. Thus two 
> forks will only be available when their two nearest neighbors are thinking, 
> not eating. After an individual philosopher finishes eating, they will put 
> down both forks

This problem came up in the concurrency module of my Computer Science course, 
so before we went through the solution in the next lecture I decided to have
a go at it myself. 

In my implementation, if a philosopher goes hungry without eating for more than
a certain amount of time then he dies and the program terminates. 

At first, after writing it out and checking that it works, I thought it might
have been cheating to only let the philosophers pick up the forks when 
both forks are free. But then I had a look at some of the other solutions, 
including Dijkstra's original one and they all control the behaviour of the 
philosophers in some way (in Dijkstra's solution a philosopher checks the 
status of his two neighbours, represented with semaphores, before attempting 
to pick up the forks).

The code for picking up both forks:

```rust
    fn eat(&mut self) {
        while let PhilosopherState::Hungry(_) = self.state {
            // Attempt to pick up both forks at the same time
            let pickup_forks =
                (self.left_fork.try_lock(), self.right_fork.try_lock());
            if let (Ok(_), Ok(_)) = pickup_forks {
                // ...
            }
        }
    }
```

The scope of the acquired mutex locks is within the scope the current iteration 
of the while loop. This means that if only one lock is acquired the loop 
continues to its next iteration and the lock goes out of scope. Once a lock 
goes out of scope, it is dropped and released. 

This might not be the most efficient implementation, because a philosopher can 
keep acquiring and releasing the lock for the same fork over and over again while 
another philosopher might also be trying to acquire that lock. Although it's 
probably not much of an issue with only two philosophers attempting to pick up 
each fork, I doubt this is a very scalable solution for applications where more
than two active processes require conditional access to a shared resource. 
However, to me at least, it was the most obvious and intuitive solution. 
