FROM openjdk:14-slim

EXPOSE 3030

ENV VERSION=3.14.0
ENV SHASUM=11cdfc60c515281efeb392394d750f1847ee77a2b0a4728ab006f4faafee916a

ADD https://downloads.apache.org/jena/binaries/apache-jena-fuseki-$VERSION.tar.gz /tmp/fuseki.tar.gz

RUN [ "$SHASUM  /tmp/fuseki.tar.gz" = "$(sha256sum /tmp/fuseki.tar.gz)" ]
RUN cd /opt && tar xvzf /tmp/fuseki.tar.gz && rm /tmp/fuseki.tar.gz

WORKDIR /opt/apache-jena-fuseki-3.14.0

ADD shiro.ini run/shiro.ini

CMD ["java", "-jar", "fuseki-server.jar"]
