classdef i_value_stack
    % I_VALUE_STACK  value-class stack
    %   Every method returns the modified object, so callers must
    %   assign the result back:  s = push(s, 3);
    %   A value class never changes the caller's copy on its own.

    properties
        Items = {};
        MaxSize = Inf;
    end

    methods
        function obj = i_value_stack(varargin)
            % constructor: any initial items are pushed in order
            for k = 1:numel(varargin)
                obj.Items{end+1} = varargin{k};
            end
        end

        function obj = push(obj, item)
            % add an item on top of the stack
            if numel(obj.Items) >= obj.MaxSize
                error('i_value_stack:full', 'stack is full');
            end
            obj.Items{end+1} = item;
        end

        function [obj, item] = pop(obj)
            % remove and return the top item
            if isEmpty(obj)
                error('i_value_stack:empty', 'stack is empty');
            end
            item = obj.Items{end};
            obj.Items(end) = [];
        end

        function item = peek(obj)
            % look at the top item without removing it
            if isEmpty(obj)
                item = [];
            else
                item = obj.Items{end};
            end
        end

        function tf = isEmpty(obj)
            % true when there is nothing on the stack
            tf = isempty(obj.Items);
        end

        function n = size(obj)
            % number of items currently stored
            n = length(obj.Items);
        end

        function obj = clear(obj)
            % empty the stack
            obj.Items = {};
        end
    end
end
