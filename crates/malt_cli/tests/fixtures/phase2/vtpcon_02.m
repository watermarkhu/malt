classdef vtpcon_02
properties
a double
b double {mustBeLessThan(b, a)} = 1
end
end
