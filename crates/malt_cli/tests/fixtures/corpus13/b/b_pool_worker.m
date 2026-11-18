classdef b_pool_worker < handle
    % R9_POOL_WORKER runs jobs from a queue and records the results.

    properties (SetAccess = private)
        Results containers.Map
        Failures double = []
    end

    properties (Constant)
        MaxRetries = 2
    end

    methods
        function obj = b_pool_worker()
            obj.Results = containers.Map('KeyType', 'char', 'ValueType', 'any');
        end

        function run(obj, jobs)
            for k = 1:numel(jobs)
                job = jobs{k};
                attempt = 0;
                done = false;
                while ~done && attempt <= obj.MaxRetries
                    attempt = attempt + 1;
                    try
                        obj.Results(job.Name) = job.Fcn(job.Input);
                        done = true;
                    catch err
                        if attempt > obj.MaxRetries
                            obj.Failures(end+1) = k;
                            fprintf('job %s failed: %s\n', job.Name, err.message)
                        end
                    end
                end
            end
        end

        function n = succeeded(obj)
            n = obj.Results.Count;
        end

        function out = summary(obj)
            names = keys(obj.Results);
            out = struct('name', names, 'value', values(obj.Results));
            out = out(:);
        end
    end
end
