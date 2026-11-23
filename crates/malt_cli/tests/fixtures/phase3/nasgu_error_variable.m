function r = nasgu_error_variable(msg)
x = 1;
try
    x = 2;
catch ME
    error(ME.message);
end
r = x;
end
