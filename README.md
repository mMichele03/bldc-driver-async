# Async BLDC motor driver

[build.rs](https://github.com/embassy-rs/embassy/blob/main/examples/rp235x/build.rs)

[memory.x](https://github.com/embassy-rs/embassy/blob/main/examples/rp235x/memory.x)

[EXAMPLES](https://github.com/embassy-rs/embassy/tree/main/examples/rp235x/src/bin)

[AS5048A magnetic encoder datasheet](https://www.mouser.com/datasheet/2/588/AS5048_DS000298_4-00-1100510.pdf?srsltid=AfmBOopVQXd4zM0YdcdOcmUoEoGPYkSAAHg5qVXJu2K3LdYZ0NbKSO3k)

## Motor parameters ([*GBM2804H-100T*](https://it.aliexpress.com/item/4001137970972.html?gatewayAdapt=glo2ita#nav-specification)) ([*Paper that uses this motor*](https://arxiv.org/pdf/2505.01740#:~:text=0.92%20(mH).%20Number%20of%20Poles.%20(P).%207.,0.07%20(N.m%2FA).%20NSGA-II%20has%20a%20computational%20%7B%22isScientific%22%3Atrue%2C%22citationCount%22%3A0%2C%22authors%22%3A%5B%5D%2C%22doi%22%3A%22%22%2C%22issuedYear%22%3A0%2C%22publisher%22%3A%22%22%2C%22containerTitle%22%3A%22%22%2C%22title%22%3A%22%22%2C%22page%22%3A%22%22%2C%22volume%22%3A%22%22%2C%22abstract%22%3A%22%22%7D))

| Parameter                    | Value                                                                            |
| ---------------------------- | -------------------------------------------------------------------------------- |
| Stator resistance ($R$)      | 5.6 Ω                                                                            |
| Stator inductance ($L$)      | 0.92 mH                                                                          |
| Back EMF coefficient ($k_e$) | 0.047 V·s/rad                                                                    |
| Friction coefficient ($B$)   | 550 nN·m·s                                                                       |
| Moment of inertia ($J$)      | 480 nN·m·s^2                                                                     |
| Number of Poles ($P$)        | 7                                                                                |
| Rated voltage ($V_{dc}$)     | 12 V                                                                             |
| Torque coefficient ($k_t$)   | 0.07 N·m/A                                                                       |
| kv rating ($KV$) ???         | 117 to 136 to [154](https://www.ebay.co.uk/itm/364946266735) to 203 to 213 rpm·V |

## Notes

- [Estimated current mode theory](https://docs.simplefoc.com/voltage_torque_control#estimated-current-mode-theory)

- [How to Build a Fixed-Point PI Controller](https://www.embeddedrelated.com/showarticle/123.php)

## `impl Steps` for FOC control algorithm
- `IN`: encoder angle

- Enstimate angle and velocity with [$\alpha-\beta$ filter](https://en.wikipedia.org/wiki/Alpha_beta_filter) (rust crate [signalo_filters](https://docs.rs/signalo_filters/0.6.1/signalo_filters/observe/alpha_beta/index.html)) or [Type-II Phase-Locked Loop](https://www.allaboutcircuits.com/technical-articles/introduction-to-second-order-type-2-plls/)
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
