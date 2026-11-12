classdef i_stack_queue_classes < handle
    % I_STACK_QUEUE_CLASSES  handle-class stack with a queue-like helper
    %   s = i_stack_queue_classes('my stack');
    %   s.push(3);
    %   x = s.pop();
    %
    %   Because it is a handle class, every method changes the
    %   object that the caller holds, so no return value is needed.

    properties
        Items = {}
        Name = 'stack'
    end

    methods
        function obj = i_stack_queue_classes(name)
            % constructor, the name is optional
            if nargin > 0
                obj.Name = name;
            end
        end

        function push(obj, item)
            % put an item on top of the stack
            obj.Items{end+1} = item;
        end

        function item = pop(obj)
            % take the top item off the stack
            if obj.isEmpty()
                error('Stack:empty', 'cannot pop from empty stack');
            end
            item = obj.Items{end};
            obj.Items(end) = [];
        end

        function item = peek(obj)
            % look at the top item without removing it
            if isEmpty(obj.Items)
                item = [];
                return;
            end
            item = obj.Items{end};
        end

        function tf = isEmpty(obj)
            % true when nothing is stored
            tf = numel(obj.Items) == 0;
        end

        function n = size(obj)
            % number of stored items
            n = numel(obj.Items);
        end

        function enqueue(obj, item)
            % queue view: add at the back
            obj.Items = [obj.Items, {item}];
        end

        function item = dequeue(obj)
            % queue view: take from the front
            if obj.isEmpty()
                error('Queue:empty', 'cannot dequeue from empty queue');
            end
            item = obj.Items{1};
            obj.Items(1) = [];
        end

        function s = describe(obj)
            % build a printable line by concatenation
            s = [obj.Name ': '];
            for k = 1:numel(obj.Items)
                s = [s num2str(obj.Items{k}) ' '];
            end
        end

        function clear(obj)
            % remove everything
            obj.Items = {};
        end
    end
end
