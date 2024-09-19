FROM postgres:15
COPY init_postgres.sh /docker-entrypoint-initdb.d/init.sh
EXPOSE 5432

CMD ["postgres"]
