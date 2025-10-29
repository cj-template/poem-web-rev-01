insert into error_stack(error_name, error_summary, error_stack, reported_at)
VALUES (:error_name, :error_summary, :error_stack, datetime());