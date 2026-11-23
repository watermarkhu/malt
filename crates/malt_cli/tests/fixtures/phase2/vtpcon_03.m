classdef vtpcon_03
properties
x double {mustBeGreaterThan(x, 0), mustBeLessThan(x, max_val)} = 1
end
end
