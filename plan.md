# Steps

1. ## Project setup
    - install rust
    - configure embassy
    - upload basic program to board

2. ## Hardware setup
    - solder driver and encoder
    - connect everything on a breadboard

3. ## Board usage example
    - simple example of reading from encoder and writing to pwm

4. ## Basic helpers
    - setup log
    - setup telemetry

5. ## Define API
    - create "hardware libraries" to control the motor and to read the sensor
      - sensor: angle
      - motor: (magnetic angle, intensity) -> (3 pwm)

6. ## Driver control algorithm
    - async actors for the specific driver control implementation
    - problem: SPI/PWM sync vs async (hardware at 25kHz)

7. ## Implement and test

8. ## Show board is agnostic: test on STM32

9. ## Deliver
