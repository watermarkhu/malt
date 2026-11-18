classdef a_queue_handle < handle
    %A_QUEUE_HANDLE A simple FIFO queue with a capacity limit.
    %   Items are stored in a cell array. When the queue is full,
    %   the oldest item is dropped and the Overflow event fires.

    properties
        Capacity (1,1) double {mustBePositive, mustBeInteger} = 10
        Label (1,:) char = 'queue'
    end

    properties (SetAccess = private)
        Items cell = {}
    end

    properties (Dependent)
        Count
        IsFull
    end

    events
        Overflow
        ItemAdded
    end

    methods
        function obj = a_queue_handle(capacity, label)
            %A_QUEUE_HANDLE Construct a queue.
            if nargin >= 1
                obj.Capacity = capacity;
            end
            if nargin >= 2
                obj.Label = label;
            end
        end

        function push(obj, item)
            %PUSH Add an item to the back of the queue.
            if obj.IsFull
                notify(obj, 'Overflow')
                obj.Items(1) = [];
            end
            obj.Items{end+1} = item;
            notify(obj, 'ItemAdded')
        end

        function item = pop(obj)
            %POP Remove and return the front item.
            if isempty(obj.Items)
                error('a_queue_handle:empty', 'Queue "%s" is empty.', obj.Label)
            end
            item = obj.Items{1};
            obj.Items(1) = [];
        end

        function item = peek(obj)
            %PEEK Return the most recently added item without removing it.
            if obj.Count == 0
                item = [];
                return
            end
            item = obj.Items{end};
        end

        function n = get.Count(obj)
            n = numel(obj.Items);
        end

        function tf = get.IsFull(obj)
            tf = obj.Count >= obj.Capacity;
        end

        function clear_all(obj)
            %CLEAR_ALL Remove every item.
            obj.Items = {};
        end
    end
end
