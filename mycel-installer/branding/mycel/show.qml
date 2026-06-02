import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import io.calamares.ui 1.0

Presentation {
    id: presentation

    Timer {
        interval: 5000
        running: true
        repeat: true
        onTriggered: presentation.goToNextSlide()
    }

    Slide {
        ColumnLayout {
            anchors.centerIn: parent
            spacing: 24

            Image {
                Layout.alignment: Qt.AlignHCenter
                source: "logo_black.png"
                width: 96
                height: 96
                fillMode: Image.PreserveAspectFit
            }
            Label {
                Layout.alignment: Qt.AlignHCenter
                text: "Welcome to MycelOS"
                font.pixelSize: 28
                font.weight: Font.Light
                color: "#cdd6f4"
            }
            Label {
                Layout.alignment: Qt.AlignHCenter
                text: "Declarative. Reproducible. Yours."
                font.pixelSize: 14
                color: "#a6adc8"
            }
        }
    }

    Slide {
        ColumnLayout {
            anchors.centerIn: parent
            spacing: 16

            Label {
                Layout.alignment: Qt.AlignHCenter
                text: "One file. Your whole system."
                font.pixelSize: 24
                font.weight: Font.Light
                color: "#cdd6f4"
            }
            Label {
                Layout.alignment: Qt.AlignHCenter
                text: "mycel.toml declares every package, service,\nand setting. Apply it with mycel switch."
                font.pixelSize: 13
                color: "#a6adc8"
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }

    Slide {
        ColumnLayout {
            anchors.centerIn: parent
            spacing: 16

            Label {
                Layout.alignment: Qt.AlignHCenter
                text: "FessusDE"
                font.pixelSize: 24
                font.weight: Font.Light
                color: "#cdd6f4"
            }
            Label {
                Layout.alignment: Qt.AlignHCenter
                text: "A lightweight Wayland desktop built for\nlow-to-mid range hardware. Fast by design."
                font.pixelSize: 13
                color: "#a6adc8"
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }

    Slide {
        ColumnLayout {
            anchors.centerIn: parent
            spacing: 16

            Label {
                Layout.alignment: Qt.AlignHCenter
                text: "Community powered."
                font.pixelSize: 24
                font.weight: Font.Light
                color: "#cdd6f4"
            }
            Label {
                Layout.alignment: Qt.AlignHCenter
                text: "Add any GitHub repo as a package overlay.\nNo server. No gatekeeping."
                font.pixelSize: 13
                color: "#a6adc8"
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }
}
