function s2(h)
import foo.*
x = 1;
import bar.*
c = onCleanup(@() disp(1));
onCleanup(@() disp(2));
[~] = onCleanup(@() disp(3));
y = eval('1');
z = 1 + eval('1');
eval('a=1');
feval(@sin, 1);
q = feval(@sin, 1);
disp(feval(@sin, 1));
warning('MATLAB:nonscalar', 'x');
warning('off', 'MATLAB:intConvertNonIntVal');
warning('MATLAB:str2func:invalidFunctionName', 'x');
warning('off', 'MATLAB:dispatcher:nameConflict');
warning('off', 'MATLAB:ode45:foo');
end
function s3()
% comment
import a.b
function_call();
import c.d
end
