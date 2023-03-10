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

# Performance Comparison
I decided to implement a few different solutions so I could test their performance 
against each other. I do two runs for each solution, one with randomness where 
thinking and eating both take a random amount of time within a certain range, 
and one without randomness where both thinking and eating take a fixed amount 
of time. 

Performance without randomness is pretty similar, but for some reason when 
some randomness is introduced the `two_forks` solution is consistently _slightly_
more efficient than the control solution and the `break symmetry` solution.
Obviously, I should run it a few more times and take an average of the results 
but I've not got round to that yet. 

I've yet to implement Dijkstra's semaphore solution and the resource hierarchy
solution. 

Sample output (with 10s runtime):
```shell
~~SEQUENTIAL (CONTROL)~~ [no randomness]
        Total meals eaten: 1608
        Philosopher 1: 322 meals
        Philosopher 2: 321 meals
        Philosopher 3: 322 meals
        Philosopher 4: 321 meals
        Philosopher 5: 322 meals

~~TWO FORKS~~ [no randomness]
        Total meals eaten: 1626
        Philosopher 1: 328 meals
        Philosopher 2: 322 meals
        Philosopher 3: 323 meals
        Philosopher 4: 325 meals
        Philosopher 5: 328 meals

~~BREAK SYMMETRY~~ [no randomness]
        Total meals eaten: 1624
        Philosopher 1: 311 meals
        Philosopher 2: 335 meals
        Philosopher 3: 334 meals
        Philosopher 4: 334 meals
        Philosopher 5: 310 meals

~~SEQUENTIAL (CONTROL)~~ [with randomness]
        Total meals eaten: 2646
        Philosopher 1: 529 meals
        Philosopher 2: 529 meals
        Philosopher 3: 530 meals
        Philosopher 4: 529 meals
        Philosopher 5: 529 meals

~~TWO FORKS~~ [with randomness]
        Total meals eaten: 2820
        Philosopher 1: 575 meals
        Philosopher 2: 547 meals
        Philosopher 3: 572 meals
        Philosopher 4: 557 meals
        Philosopher 5: 569 meals

~~BREAK SYMMETRY~~ [with randomness]
        Total meals eaten: 2782
        Philosopher 1: 512 meals
        Philosopher 2: 639 meals
        Philosopher 3: 580 meals
        Philosopher 4: 557 meals
        Philosopher 5: 494 meals
```