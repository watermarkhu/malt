function varargout = g_vararg_rhs(varargin)
% The right-hand side decides: literals and non-cell calls are VARARG.
x = varargin;
varargout = x;
end

function varargout = g_vararg_literal(varargin) %#ok<INUSD>
varargout = 0;
end

function varargout = g_vararg_call(varargin) %#ok<INUSD>
varargout = ones(2);
end

function varargout = g_vararg_op(varargin)
varargout = varargin(1);
end

function varargout = g_vararg_cell(varargin) %#ok<INUSD>
varargout = {1};
end
