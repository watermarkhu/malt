function [out,err]=k_parens_style(x,mode)
%K_PARENS_STYLE clamps and scales values
%   out=k_parens_style(x,mode) scales x by a factor chosen from mode.
%   Written by someone who likes parentheses everywhere.
if(nargin<2)
mode='default';
end
if(nargin<1)||isempty(x)
out=[];err='no input';
return
end
err='';
out=zeros(size(x));
for(i=1:numel(x))
if(x(i)>3)
out(i)=x(i)*2;
elseif(x(i)<(-3))
out(i)=-x(i);
else
out(i)=x(i);
end
end
switch(mode)
case{'default','plain'}
factor=1;
case{'double'}
factor=2;
case 'half'
factor=0.5;
otherwise
error('k:mode','unknown mode %s',mode);
end
out=out*factor;
while(true)
if(all(abs(out)<1e6))
break
end
out=out/10;
end
[~,idx]=max(out);
if(idx>0)&&(isfinite(out(idx)))
err=sprintf('max at %d',idx);
end
f=@(v)v.^2;
out=f(out);
s=struct('a',1,'b',2);
names=fieldnames(s);
for(k=1:numel(names))
disp (names{k})
end
disp ('done')
if(~isempty(names)&&(numel(names)>=2))
fprintf('%d fields\n',numel(names));
end
tmp=[1,2,3,];
total=sum(tmp(:))+sum(out(:));
if(total>=0)
err=[err,' ok'];
else
err=[err,' negative'];
end
end
