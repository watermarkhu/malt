function out = display(x)
  out = x;
end
function t7b()
  try
    a = 1;
  catch, disp(1)
  end
  try
    a = 1;
  catch %comment
    disp(1)
  end
  try
    a = 1;
  catch e, disp(e)
  end
  try
    a = 1;
  catch
  end
end
