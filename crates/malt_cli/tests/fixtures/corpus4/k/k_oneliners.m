% k_oneliners.m  -- script, quick and dirty, written in a hurry
clear; close all; clc
a=1;b=2;c=a+b;d=c*2
x=1:10; y=x.^2; z=sum(y); w=mean(y)
if a<b, disp('a is smaller'), end
if a>b, disp('no'); else, disp('yes'); end
for i=1:3, fprintf('%d\n',i); end
for j=[1 2 3], s=j; end, s
k=0; while k<5, k=k+1; end, k
n=0; do_something=true;
if do_something, n=n+1;, end
try, error('boom'), catch err, disp(err.message), end
switch a, case 1, disp('one'), otherwise, disp('other'), end
f=@(t)t.^2+1; g=@(u,v)u+v;
m=[1,2,3,]; p=[ 1 , 2 ; 3 , 4 ];
q=m'; r=p(1,:); s2=p( 2 , : );
r1=max(m); [~,idx]=max(m);
if idx==3, fprintf('last\n'), end
str="hello"; ch='world';
c2={1,'two',[3 4]};
v = ...
  3;
tbl=table([1;2],["a";"b"],'VariableNames',{'id','name'})
t0=datetime(2024,1,1); dt=t0+days(3)
out=twice(v); out2=g(1,2); out3=f(2)
for q2=1:2, for q3=1:2, fprintf('%d,%d ',q2,q3); end, end, fprintf('\n')
A=eye(2);B=A*2;C=A+B; D=C-A; E=D/2; F=E';
if isempty(ch), ch='none'; end; ch(1)
u=-1; while u<0, u=u+0.5; end
x(end+1)=11; y(end)=[]; numel(x), numel(y)
res=cellfun(@(c)numel(c),c2); lens=length(c2)
ok=isequal(size(A),[2 2]) && all(A(:)>=0)
w2 = int32(7); w3 = w2/2
z1 = {}; z1{end+1}='a'; z1{end+1}='b';
st.name='demo'; st.value=42; st
cnt=0; for ii=1:4, if mod(ii,2)==0, cnt=cnt+1; end, end, cnt
msg=sprintf('%d items',cnt); disp(msg)
lst=strsplit('a,b,c',','); numel(lst)
function out = twice(in)
out = 2*in;
end
