FROM alpine:3.15


# wendy build

RUN apk add --no-cache \
    nodejs=16.20.1-r0 \
    npm=8.1.3-r0 

ARG APP_ENV
ARG VOL_OUT

COPY .env /opt/app/
COPY package.json /opt/app/package.json
COPY package-lock.json /opt/app/package-lock.json
COPY prisma /opt/app/prisma
COPY samples /opt/app/samples
COPY src /opt/app/src

WORKDIR /opt/app/

ENV AUDIO_DEST=${VOL_OUT:-out}
# ENV NODE_ENV=${APP_ENV:-local}

# VOLUME $AUDIO_DEST:/opt/app/out
RUN npm clean-install
RUN echo "testerr $(ls -al .)"

RUN npm install -g prisma ts-node
RUN npx prisma generate  


# supercollider build 
RUN apk add --no-cache \
    alsa-lib-dev=1.2.5.1-r1 \
    boost-dev=1.77.0-r1  \
    boost-static  \
    cmake=3.21.7-r0  \
    git=2.34.8-r0 \
    jack=1.9.19-r2 \
    jack-dev=1.9.19-r2 \
    eudev-dev=3.2.11_pre1-r0  \
    fftw-dev=3.3.10-r0  \
    g++=10.3.1_git20211027-r0 \
    lame=3.100-r0  \
    libsndfile-dev=1.0.31-r1  \
    libxt-dev=1.2.1-r0  \
    linux-headers=5.10.41-r0  \
    make=4.3-r0 \
    ncurses-dev=6.3_p20211120-r2  \
    portaudio-dev=19.7.0-r0  \
    readline-dev=8.1.1-r0  \
    samurai=1.2-r1  \
    patch=2.7.6-r7 \
    vim=8.2.4836-r0 \
    yaml-cpp-dev=0.6.3-r1

ARG SC_BRANCH=3.12

WORKDIR /root/

COPY sc-alpine.patch .
ENV QT_QPA_PLATFORM=offscreen


RUN git clone \
    --depth 1 \
    --branch $SC_BRANCH \
    https://github.com/SuperCollider/SuperCollider.git && \
    cd SuperCollider && \
    # apply submodule fix
    # see https://github.com/supercollider/supercollider/issues/5695#issuecomment-1072263846
    sed -i "s|git://|https://|g" .gitmodules && \
    git submodule update --init --recursive && \
    mkdir -p /root/SuperCollider/build && \
    # apply patch for alpine, see
    # https://github.com/supercollider/supercollider/issues/5197#issuecomment-1047188442
    patch < /root/sc-alpine.patch && \
    cd /root/SuperCollider/build && \
    cmake \
        -DCMAKE_BUILD_TYPE=Debug \
        -DSUPERNOVA=OFF \
        -DSC_ED=OFF \
        -DSC_EL=OFF \
        -DSC_VIM=ON \
        -DNATIVE=ON \
        -DSC_IDE=OFF \
        -DNO_X11=ON \
        -DSC_ABLETON_LINK=OFF \
        -DSC_QT=OFF .. && \
	cmake --build . --config Debug --target all -j${MAKE_JOBS} && \
    cmake --build . --config Debug --target install -j${MAKE_JOBS} && \
    rm -rf /root/SuperCollider

WORKDIR /opt/app/
ENTRYPOINT [ "ts-node", "src/index.ts" ]