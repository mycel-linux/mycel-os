import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.0 as Kirigami
import io.calamares.ui 1.0

Page {
    id: root

    property string selectedBrowser: "firefox"
    property string selectedShell: "bash"

    Component.onCompleted: {
        Calamares.globalStorage.insert("selectedBrowser", selectedBrowser)
        Calamares.globalStorage.insert("selectedShell", selectedShell)
    }

    function onLeave() {
        Calamares.globalStorage.insert("selectedBrowser", selectedBrowser)
        Calamares.globalStorage.insert("selectedShell", selectedShell)
    }

    ScrollView {
        anchors.fill: parent

        ColumnLayout {
            width: parent.width
            spacing: 32

            // Browser section
            ColumnLayout {
                Layout.fillWidth: true
                Layout.leftMargin: 48
                Layout.rightMargin: 48
                spacing: 16

                Label {
                    text: "Choose a browser"
                    font.pixelSize: 20
                    font.weight: Font.Medium
                    color: "#ffffff"
                }

                RowLayout {
                    spacing: 16
                    Layout.fillWidth: true

                    Repeater {
                        model: [
                            { id: "firefox",    name: "Firefox",    desc: "Fast, private, by Mozilla" },
                            { id: "librewolf",  name: "LibreWolf",  desc: "Firefox, hardened for privacy" },
                            { id: "zen-browser",name: "Zen",        desc: "Beautiful Firefox fork" }
                        ]

                        delegate: Rectangle {
                            Layout.fillWidth: true
                            height: 100
                            radius: 10
                            color: root.selectedBrowser === modelData.id ? "#3F549E" : "#1e1e2e"
                            border.color: root.selectedBrowser === modelData.id ? "#6070be" : "#313244"
                            border.width: 2

                            ColumnLayout {
                                anchors.centerIn: parent
                                spacing: 6

                                Label {
                                    Layout.alignment: Qt.AlignHCenter
                                    text: modelData.name
                                    font.pixelSize: 15
                                    font.weight: Font.Medium
                                    color: "#ffffff"
                                }
                                Label {
                                    Layout.alignment: Qt.AlignHCenter
                                    text: modelData.desc
                                    font.pixelSize: 11
                                    color: "#a6adc8"
                                    wrapMode: Text.WordWrap
                                    horizontalAlignment: Text.AlignHCenter
                                }
                            }

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: {
                                    root.selectedBrowser = modelData.id
                                    Calamares.globalStorage.insert("selectedBrowser", modelData.id)
                                }
                            }
                        }
                    }
                }
            }

            // Shell section
            ColumnLayout {
                Layout.fillWidth: true
                Layout.leftMargin: 48
                Layout.rightMargin: 48
                spacing: 16

                Label {
                    text: "Choose a shell"
                    font.pixelSize: 20
                    font.weight: Font.Medium
                    color: "#ffffff"
                }

                RowLayout {
                    spacing: 16
                    Layout.fillWidth: true

                    Repeater {
                        model: [
                            { id: "bash", name: "Bash",  desc: "Universal, always available" },
                            { id: "zsh",  name: "Zsh",   desc: "Powerful with great plugins" },
                            { id: "fish", name: "Fish",  desc: "Friendly, works out of the box" }
                        ]

                        delegate: Rectangle {
                            Layout.fillWidth: true
                            height: 100
                            radius: 10
                            color: root.selectedShell === modelData.id ? "#3F549E" : "#1e1e2e"
                            border.color: root.selectedShell === modelData.id ? "#6070be" : "#313244"
                            border.width: 2

                            ColumnLayout {
                                anchors.centerIn: parent
                                spacing: 6

                                Label {
                                    Layout.alignment: Qt.AlignHCenter
                                    text: modelData.name
                                    font.pixelSize: 15
                                    font.weight: Font.Medium
                                    color: "#ffffff"
                                }
                                Label {
                                    Layout.alignment: Qt.AlignHCenter
                                    text: modelData.desc
                                    font.pixelSize: 11
                                    color: "#a6adc8"
                                    wrapMode: Text.WordWrap
                                    horizontalAlignment: Text.AlignHCenter
                                }
                            }

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: {
                                    root.selectedShell = modelData.id
                                    Calamares.globalStorage.insert("selectedShell", modelData.id)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
