classdef b_interval_class
    properties
        Lo
        Hi
    end
    methods
        function obj = b_interval_class(lo, hi)
            if nargin == 2
                obj.Lo = lo;
                obj.Hi = hi;
            end
        end
        function r = plus(a, b)
            if isnumeric(b)
                r = b_interval_class(a.Lo + b, a.Hi + b);
            else
                r = b_interval_class(a.Lo + b.Lo, a.Hi + b.Hi);
            end
        end
        function r = mtimes(a, k)
            r = b_interval_class(a.Lo * k, a.Hi * k);
            if k < 0
                tmp = r.Lo;
                r.Lo = r.Hi;
                r.Hi = tmp;
            end
        end
        function tf = contains(obj, x)
            tf = x >= obj.Lo & x <= obj.Hi;
        end
        function w = width(obj)
            w = obj.Hi - obj.Lo;
        end
        function varargout = subsref(obj, s)
            switch s(1).type
                case '.'
                    [varargout{1:nargout}] = builtin('subsref', obj, s);
                case '()'
                    idx = s(1).subs{1};
                    vals = [obj.Lo obj.Hi];
                    varargout{1} = vals(idx);
                otherwise
                    error('interval:badIndex', 'Unsupported index type');
            end
        end
        function disp(obj)
            disp(['[' num2str(obj.Lo) ', ' num2str(obj.Hi) ']']);
        end
    end
end
