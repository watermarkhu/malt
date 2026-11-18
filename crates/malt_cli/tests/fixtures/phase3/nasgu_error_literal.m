function r = nasgu_error_literal()
x = 1;
try
    x = 2;
catch ME
    error('b');
end
r = x;
end
