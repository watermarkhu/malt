classdef i_customer
    % I_CUSTOMER  a value class that holds account handles in a cell array
    %   Because it is a value class, addAccount returns a new object:
    %   c = c.addAccount(acc);

    properties
        Name = ''
        Accounts = {}
        Created = ''
    end

    methods
        function obj = i_customer(name)
            % store the name and a creation stamp
            if nargin < 1
                name = 'anonymous';
            end
            obj.Name = name;
            obj.Created = datestr(now);
        end

        function obj = addAccount(obj, acct)
            % append an account to the list
            obj.Accounts{end+1} = acct;
        end

        function total = totalBalance(obj)
            % sum the balances of every account
            total = 0;
            for i = 1:numel(obj.Accounts)
                total = total + obj.Accounts{i}.Balance;
            end
        end

        function n = numAccounts(obj)
            % how many accounts the customer has
            n = length(obj.Accounts);
        end

        function acct = findAccount(obj, owner_name)
            % return the first account whose owner matches, or []
            acct = [];
            for i = 1:numel(obj.Accounts)
                if strcmp(obj.Accounts{i}.Owner, owner_name)
                    acct = obj.Accounts{i};
                    return;
                end
            end
        end

        function obj = removeAccount(obj, idx)
            % drop the account at position idx
            if idx < 1 || idx > numel(obj.Accounts)
                error('i_customer:index', 'no account %d', idx);
            end
            obj.Accounts(idx) = [];
        end

        function report(obj)
            % print every account
            fprintf('Customer: %s (since %s)\n', obj.Name, obj.Created);
            for i = 1:numel(obj.Accounts)
                disp(['  ' obj.Accounts{i}.summary()]);
            end
        end
    end
end
