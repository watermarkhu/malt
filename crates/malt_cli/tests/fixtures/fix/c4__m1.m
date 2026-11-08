function m1(n)
parfor i = 1:n
  r = r + i;
  G = 1;
end
end
function m2(n)
global G
G = 0;
end
function m3(n)
G = 0;
parfor i = 1:n
  G = G + i;
end
end
function validateFoo(a, b)
validateattributes(b, {'numeric'}, {});
end
function validateBar(a, b)
validateattributes(a, {'numeric'}, {});
end
function validateBaz(a)
mustBePositive(a.x);
end
function o = loc(q)
o.a = 1;
q.b = 2;
end
function imp()
import a.*
import b.*
x = 1;
end
function r = outer()
for k = 1:3
  inner();
end
  function inner()
    k = 5;
    for k = 1:2, end
  end
  r = 1;
end
function z = ctor()
z = foo(foo(1));
z = foo(bar(1));
z = foo.bar(foo(1));
z = Foo(Foo());
end
