classdef l_vector2
    %L_VECTOR2 Two-dimensional vector with arithmetic operators.
    %   Vectors are values: operators return new objects and never
    %   modify their inputs. Indexing with () returns a component.
    %
    %   Example:
    %       v = l_vector2(3, 4);
    %       w = v * 2 - l_vector2(1, 1);
    %       w.Norm

    properties
        X (1,1) double = 0
        Y (1,1) double = 0
    end

    properties (Dependent)
        Norm
    end

    methods
        function obj = l_vector2(x, y)
            if nargin == 0
                return
            end
            obj.X = x;
            obj.Y = y;
        end

        function n = get.Norm(obj)
            n = hypot(obj.X, obj.Y);
        end

        function r = plus(a, b)
            r = l_vector2(a.X + b.X, a.Y + b.Y);
        end

        function r = minus(a, b)
            r = l_vector2(a.X - b.X, a.Y - b.Y);
        end

        function r = times(a, b)
            if isnumeric(a)
                r = l_vector2(a * b.X, a * b.Y);
            else
                r = l_vector2(a.X * b, a.Y * b);
            end
        end

        function tf = eq(a, b)
            tf = a.X == b.X & a.Y == b.Y;
        end

        function tf = isequal(a, b)
            tf = isa(b, 'l_vector2') && isequal([a.X a.Y], [b.X b.Y]);
        end

        function disp(obj)
            fprintf('(%g, %g)\n', obj.X, obj.Y);
        end

        function n = numel(obj, varargin)
            n = 1;
        end

        function varargout = size(~, dim)
            sz = [1 1];
            if nargin == 2
                varargout{1} = sz(dim);
            else
                varargout{1} = sz;
            end
        end

        function r = withX(obj, x)
            obj.X = x;
        end

        function varargout = subsref(obj, s)
            switch s(1).type
                case '.'
                    varargout{1} = obj.(s(1).subs);
                case '()'
                    idx = s(1).subs{1};
                    varargout{1} = obj.X * (idx == 1) + obj.Y * (idx == 2);
                otherwise
                    error('l_vector2:badIndex', 'Unsupported index type %s.', s(1).type);
            end
        end

        function r = normalised(obj)
            n = obj.Norm;
            if n == 0
                r = obj;
                return
            end
            r = l_vector2(obj.X / n, obj.Y / n);
        end
    end
end
