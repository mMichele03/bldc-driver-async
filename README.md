# Async BLDC motor driver

[build.rs](https://github.com/embassy-rs/embassy/blob/main/examples/rp235x/build.rs)

[memory.x](https://github.com/embassy-rs/embassy/blob/main/examples/rp235x/memory.x)

[EXAMPLES](https://github.com/embassy-rs/embassy/tree/main/examples/rp235x/src/bin)

[AS5048A magnetic encoder datasheet](https://www.mouser.com/datasheet/2/588/AS5048_DS000298_4-00-1100510.pdf?srsltid=AfmBOopVQXd4zM0YdcdOcmUoEoGPYkSAAHg5qVXJu2K3LdYZ0NbKSO3k)

## Motor parameters ([*GBM2804H-100T*](https://it.aliexpress.com/item/4001137970972.html?gatewayAdapt=glo2ita#nav-specification))

| Parameter                    | Value         |
| ---------------------------- | ------------- |
| Stator resistance ($R$)      | 5.6 Ω         |
| Stator inductance ($L$)      | 0.92 mH       |
| Back EMF coefficient ($k_e$) | 0.047 V·s/rad |
| Friction coefficient ($B$)   | 550 nN·m·s    |
| Moment of inertia ($J$)      | 480 nN·m·s^2  |
| Number of Poles ($P$)        | 7             |
| Rated voltage ($V_{dc}$)     | 12 V          |
| Torque coefficient ($k_t$)   | 0.07 N·m/A    |

## `impl Steps` for FOC control algorithm
- `IN`: encoder angle

- Enstimate angle and velocity with [$\alpha-\beta$ filter](https://en.wikipedia.org/wiki/Alpha_beta_filter) (rust crate [signalo_filters](https://docs.rs/signalo_filters/0.6.1/signalo_filters/observe/alpha_beta/index.html))
    $$ \hat{\theta}_k, \hat{\omega}_k $$

- Find the feed-forward voltage $V_{ff}$: 
    $$ V_{ff} = k_e · \omega_{???} $$

- Set direct and quadrature voltage $V_d, V_q$:
    $$ V_d = 0 \\ V_q = PI(\omega_{target} - \hat{\omega}_k) + V_{ff} $$

- Find PWMs from voltage (rust crate [fluxkit_math](https://crates.io/crates/fluxkit_math))

  - Inverse [Park Transform](https://it.mathworks.com/help/sps/ref/parktransform.html)

  - Inverse [Clarke Transform](https://it.mathworks.com/help/sps/ref/clarketransform.html)

  - Find DC with Space Vector Modulation ([SVM or SVPWM](https://it.mathworks.com/discovery/space-vector-modulation.html))

- `OUT`: set PWMs
