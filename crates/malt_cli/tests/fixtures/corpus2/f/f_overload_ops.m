classdef f_overload_ops
%F_OVERLOAD_OPS Small 2-D point type with operator overloading.
%   V = f_overload_ops([1 2; 3 4]) creates two points. Arithmetic, indexing
%   and display follow the usual MATLAB conventions.

    properties
        Data (:,2) double = zeros(0,2)
    end

    properties (Dependent)
        Magnitude
    end

    methods
        function obj = f_overload_ops(data)
            if nargin > 0
                obj.Data = reshape(data, [], 2);
            end
        end

        function m = get.Magnitude(obj)
            m = sqrt(sum(obj.Data.^2, 2));
        end

        function c = plus(a, b)
            c = f_overload_ops(a.Data + b.Data);
        end

        function c = minus(a, b)
            c = f_overload_ops(a.Data - b.Data);
        end

        function c = mtimes(a, s)
            % Scalar multiplication only
            c = f_overload_ops(a.Data * s);
        end

        function n = numel(obj, varargin)
            n = size(obj.Data, 1);
        end

        function varargout = size(obj, varargin)
            [varargout{1:nargout}] = size(obj.Data, varargin{:});
        end

        function out = subsref(obj, s)
            switch s(1).type
                case '()'
                    out = obj.Data(s(1).subs{:});
                case '.'
                    out = obj.(s(1).subs);
                otherwise
                    error('f_overload_ops:badIndex', 'Unsupported indexing.');
            end
            if numel(s) > 1
                out = subsref(out, s(2:end));
            end
        end

        function obj = subsasgn(obj, s, val)
            if strcmp(s(1).type, '()')
                obj.Data(s(1).subs{:}) = val;
            else
                error('f_overload_ops:badAssign', 'Only () assignment is supported.');
            end
        end

        function c = horzcat(varargin)
            % Joining points stacks their rows
            allData = cellfun(@(v) v.Data, varargin, 'UniformOutput', false);
            c = f_overload_ops(vertcat(allData{:}));
        end

        function disp(obj)
            fprintf('f_overload_ops with %d points\n', size(obj.Data, 1));
            disp(obj.Data);
        end

        function display(obj)
            fprintf('%s =\n', inputname(1));
            disp(obj);
        end
    end
end
