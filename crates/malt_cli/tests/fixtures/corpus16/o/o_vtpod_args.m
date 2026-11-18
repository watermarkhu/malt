function o_vtpod_args(a, b)
    arguments
        a double (1,1) {mustBePositive} = 1
        b (1,1) double {mustBePositive} = 1
    end
    disp(a + b);
end
