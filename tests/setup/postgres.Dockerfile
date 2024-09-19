FROM postgres:13
WORKDIR /
RUN pwd
COPY init_postgres.sh /docker-entrypoint-initdb.d/init.sh
RUN chmod +x /docker-entrypoint-initdb.d/init.sh
EXPOSE 5432

CMD ["postgres"]
