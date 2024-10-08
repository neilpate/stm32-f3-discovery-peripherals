# stm32-f3-discovery-barebones
The most basic, (almost) dependency free example possible to start STM32 tinkering in rust with the F3 Discovery board. (https://www.st.com/en/evaluation-tools/stm32f3discovery.html)

Most of the embedded rust tutorials start with the HAL type crates, and while these work fine they bring a lot of baggage with them. This makes it hard to understand exactly what is necessary to get started.

The intention of this repo is to try and provide an as simple as possible example.

It is assumed that VSCode will be used as the editor.

# Requirements
- cortex-debug extension for VSCode
- OpenOCD
- GDB for ARM

# Running it!
Open the project in VSCode and hit F5.

# How does the build chain work?
When run from VSCode, the binary is built and then the cortex-debug extension (which includes a runner which automatically starts OpenOCD and connects to it using GDB) will automatically load the code onto the target.

# What should I expect it to do?
When you press the User button the West LED (Green) will light.
The switch is connected to Port A.0
The LED is connected to Port E.15

# Schematic 

![image](https://github.com/user-attachments/assets/07dcba22-c1b5-420b-87cf-1f5f03ab0944)

![image](https://github.com/user-attachments/assets/98ebd84d-acb2-41db-8b02-55962e2c9b04)


