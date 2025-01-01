> [!CAUTION]
> # Project archived
> The project has been archived due to the aero2 service no longer utilizing a captcha.
> 
> Here is the announcement made by Polkomtel:
> > Szanowni Państwo,
> >
> > Polkomtel Sp. z o.o. informuje, że z dniem 30 grudnia 2024 r. zakończy udostępnianie Bezpłatnego Dostępu do Internetu (dalej: BDI). Działanie to jest podyktowane wygaśnięciem obowiązku udostępniania BDI, o którym mowa w Decyzji Prezesa UKE nr DZC-WAP-5176-8/09 (21) z dnia 10 listopada 2009 r. zmienianej kolejnymi decyzjami.
> >
> > Szczegółowe informacje w tym zakresie na trwałym nośniku pod bezpiecznym linkiem:
> > https://api.plus.pl/tos/e67cb60448cc137ff40b515dbe952b35d957ec80a705707623131b08da4a710f.pdf

# aero2solver

Solve Aero2 captchas automatically using the magic of machine learning and computer vision.

## Usage

This project is intended to be run as a docker container. 
Prebuilt images are available on [Docker Hub](https://hub.docker.com/r/dumbaspl/aero2solver).

### Requirements

- Docker (or any other OCI compatible container runtime)
- The container needs to be able to resolve and connect to [http://bdi.free.aero2.net.pl:8080/](http://bdi.free.aero2.net.pl:8080/).

This project doesn't need any GPU acceleration. 
Even on low end hardware the solving speed is fairly quick because the captchas are small and infrequent.

### Running

you can test it by running:
```bash
docker run -it --rm dumbaspl/aero2solver
```

show all available options:
```bash
docker run -it --rm dumbaspl/aero2solver --help
```

run as a daemon that starts on boot:
```bash
docker run -d --restart=always --name aero2solver dumbaspl/aero2solver
```

### Running on RouterOS

It is also possible to run this container on RouterOS using the [container](https://help.mikrotik.com/docs/display/ROS/Container) package.
This removes the need for a separate machine to run the solver on.

## Information

The training data was collected by marking up 500 training + 100 validation captchas by hand.
The model was trained for 20000 iterations starting from the `yolov4-tiny` pre-trained weights.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Aero2](https://aero2.pl/) for providing "free" internet access :wink:
- [darknet-rust](https://github.com/alianse777/darknet-rust) for providing a Rust wrapper for [darknet](https://github.com/AlexeyAB/darknet)
- ~~Slaves~~ Friends for marking up all the training data
