import { useState } from "react";
import { Modal, Pressable, ScrollView, StyleSheet, Text, TouchableOpacity, View } from "react-native";
import { colors } from "@/constants/theme";

type ClassItem = {
  id: string;
  time: string;
  name: string;
  instructor: string;
  capacity: number;
  userJoined: boolean;
  attendees: string[];
};

const INITIAL_CLASSES: ClassItem[] = [
  {
    id: "c1",
    time: "6:00 AM",
    name: "Crossfit",
    instructor: "Coach Tom",
    capacity: 12,
    userJoined: false,
    attendees: ["Alex", "Jordan", "Sam", "Taylor", "Casey", "Morgan", "Riley", "Jamie", "Drew"],
  },
  {
    id: "c2",
    time: "7:30 AM",
    name: "Sunnyside Strength and Conditioning",
    instructor: "Coach Tom",
    capacity: 14,
    userJoined: false,
    attendees: [
      "Alex",
      "Jordan",
      "Sam",
      "Taylor",
      "Casey",
      "Morgan",
      "Riley",
      "Jamie",
      "Drew",
      "Avery",
      "Quinn",
      "Reese",
      "Skyler",
      "Rowan",
    ],
  },
  {
    id: "c3",
    time: "9:00 AM",
    name: "Crossfit",
    instructor: "Coach Tom",
    capacity: 15,
    userJoined: true,
    attendees: ["Jordan", "Sam", "Taylor", "Casey", "Morgan", "Riley", "Jamie", "Drew", "Avery", "You"],
  },
  {
    id: "c4",
    time: "12:00 PM",
    name: "Crossfit",
    instructor: "Coach Tom",
    capacity: 10,
    userJoined: false,
    attendees: ["Alex", "Jordan", "Sam", "Taylor", "Casey", "Morgan", "Riley", "Jamie"],
  },
  {
    id: "c5",
    time: "4:00 PM",
    name: "Crossfit",
    instructor: "Coach Alex",
    capacity: 14,
    userJoined: false,
    attendees: ["Alex", "Jordan", "Sam", "Taylor", "Casey"],
  },
  {
    id: "c6",
    time: "5:15 PM",
    name: "Crossfit",
    instructor: "Coach Alex",
    capacity: 14,
    userJoined: false,
    attendees: ["Alex", "Jordan", "Sam", "Taylor", "Casey"],
  },
  {
    id: "c7",
    time: "6:30 PM",
    name: "Crossfit",
    instructor: "Coach Alex",
    capacity: 14,
    userJoined: false,
    attendees: ["Alex", "Jordan", "Sam", "Taylor", "Casey"],
  },
];

const days = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

export default function Classes() {
  const [classes, setClasses] = useState<ClassItem[]>(INITIAL_CLASSES);
  const [selectedClassId, setSelectedClassId] = useState<string | null>(null);

  const date = new Date();
  const selectedClass = classes.find((c) => c.id === selectedClassId) ?? null;

  function toggleJoin(id: string) {
    setClasses((prev) =>
      prev.map((c) => {
        if (c.id !== id) return c;
        if (c.userJoined) {
          return { ...c, userJoined: false, attendees: c.attendees.filter((n) => n !== "You") };
        }
        if (c.attendees.length >= c.capacity) return c;
        return { ...c, userJoined: true, attendees: [...c.attendees, "You"] };
      }),
    );
  }

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.heading}>Classes</Text>
        <Text style={styles.subheading}>
          {`${days[date.getDay()]}, ${date.getMonth() + 1}/${date.getDate()}`} · reserve your spot
        </Text>
      </View>

      <ScrollView contentContainerStyle={styles.content}>
        {classes.map((item) => {
          const filled = item.attendees.length;
          const spotsLeft = item.capacity - filled;
          const full = spotsLeft <= 0;
          const lowSpots = !full && spotsLeft <= 2;
          const percent = Math.round((filled / item.capacity) * 100);
          const spotsLabel = full ? "Full" : `${spotsLeft} left`;
          const spotsColor = full ? colors.muted : lowSpots ? colors.danger : colors.accent;

          return (
            <View key={item.id} style={styles.card}>
              <View style={styles.timeChip}>
                <Text style={styles.timeChipText}>{item.time}</Text>
              </View>
              <Text style={styles.cardTitle}>{item.name}</Text>
              <Text style={styles.instructor}>{item.instructor}</Text>

              <View style={styles.progressTrack}>
                <View style={[styles.progressFill, { width: `${percent}%` }]} />
              </View>
              <View style={styles.spotsRow}>
                <Text style={styles.spotsFilled}>
                  {filled}/{item.capacity} spots filled
                </Text>
                <Text style={[styles.spotsLabel, { color: spotsColor }]}>{spotsLabel}</Text>
              </View>

              <View style={styles.actionsRow}>
                <TouchableOpacity
                  onPress={() => setSelectedClassId(item.id)}
                  accessibilityLabel={`See who's signed up for ${item.name}`}
                >
                  <Text style={styles.rosterLink}>See who&apos;s coming ({filled})</Text>
                </TouchableOpacity>

                {item.userJoined ? (
                  <TouchableOpacity style={styles.cancelButton} onPress={() => toggleJoin(item.id)}>
                    <Text style={styles.cancelButtonText}>Cancel</Text>
                  </TouchableOpacity>
                ) : full ? (
                  <View style={styles.fullButton}>
                    <Text style={styles.fullButtonText}>Full</Text>
                  </View>
                ) : (
                  <TouchableOpacity style={styles.joinButton} onPress={() => toggleJoin(item.id)}>
                    <Text style={styles.joinButtonText}>Sign Up</Text>
                  </TouchableOpacity>
                )}
              </View>
            </View>
          );
        })}
      </ScrollView>

      <Modal
        visible={selectedClass !== null}
        transparent
        animationType="fade"
        onRequestClose={() => setSelectedClassId(null)}
      >
        <View style={styles.modalOverlay}>
          <Pressable
            style={StyleSheet.absoluteFill}
            accessibilityLabel="Close roster"
            onPress={() => setSelectedClassId(null)}
          />
          {selectedClass && (
            <View style={styles.sheet}>
              <View style={styles.sheetHeader}>
                <View>
                  <Text style={styles.sheetTitle}>{selectedClass.name}</Text>
                  <Text style={styles.sheetSubtitle}>
                    {selectedClass.time} · {selectedClass.attendees.length}/{selectedClass.capacity} signed up
                  </Text>
                </View>
                <TouchableOpacity
                  style={styles.closeButton}
                  onPress={() => setSelectedClassId(null)}
                  accessibilityLabel="Close"
                >
                  <Text style={styles.closeButtonText}>✕</Text>
                </TouchableOpacity>
              </View>

              <ScrollView style={styles.rosterList}>
                {selectedClass.attendees.map((name, i) => {
                  const isYou = name === "You";
                  return (
                    <View key={`${name}-${i}`} style={styles.rosterRow}>
                      <View style={[styles.avatar, isYou && styles.avatarYou]}>
                        <Text style={[styles.avatarText, isYou && styles.avatarTextYou]}>
                          {isYou ? "Y" : name.charAt(0).toUpperCase()}
                        </Text>
                      </View>
                      <Text style={[styles.rosterName, isYou && styles.rosterNameYou]}>{name}</Text>
                    </View>
                  );
                })}
              </ScrollView>
            </View>
          )}
        </View>
      </Modal>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: colors.background,
  },
  content: {
    paddingHorizontal: 20,
    paddingTop: 16,
    paddingBottom: 40,
  },
  header: {
    paddingHorizontal: 20,
    paddingTop: 60,
    paddingBottom: 16,
    backgroundColor: colors.background,
    borderBottomWidth: 1,
    borderBottomColor: colors.border,
    zIndex: 1,
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.3,
    shadowRadius: 4,
    elevation: 3,
  },
  heading: {
    fontSize: 28,
    fontWeight: "700",
    color: colors.text,
  },
  subheading: {
    fontSize: 14,
    color: colors.muted,
    marginTop: 4,
  },
  card: {
    backgroundColor: colors.card,
    borderRadius: 14,
    padding: 16,
    marginBottom: 12,
  },
  timeChip: {
    alignSelf: "flex-start",
    backgroundColor: colors.accent2,
    borderRadius: 6,
    paddingHorizontal: 8,
    paddingVertical: 4,
    marginBottom: 6,
  },
  timeChipText: {
    color: colors.onAccent,
    fontSize: 11,
    fontWeight: "700",
    letterSpacing: 0.3,
  },
  cardTitle: {
    fontSize: 16,
    fontWeight: "700",
    color: colors.text,
  },
  instructor: {
    fontSize: 13,
    color: colors.muted,
    marginTop: 2,
    marginBottom: 10,
  },
  progressTrack: {
    height: 6,
    backgroundColor: colors.cardAlt,
    borderRadius: 3,
    overflow: "hidden",
  },
  progressFill: {
    height: 6,
    backgroundColor: colors.accent,
    borderRadius: 3,
  },
  spotsRow: {
    flexDirection: "row",
    justifyContent: "space-between",
    marginTop: 6,
  },
  spotsFilled: {
    fontSize: 13,
    color: colors.muted,
  },
  spotsLabel: {
    fontSize: 13,
    fontWeight: "700",
  },
  actionsRow: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
    marginTop: 12,
  },
  rosterLink: {
    fontSize: 13,
    color: colors.muted,
    textDecorationLine: "underline",
  },
  joinButton: {
    backgroundColor: colors.accent2,
    borderRadius: 8,
    paddingHorizontal: 16,
    paddingVertical: 9,
  },
  joinButtonText: {
    color: colors.onAccent,
    fontSize: 13,
    fontWeight: "600",
  },
  cancelButton: {
    backgroundColor: colors.surface,
    borderWidth: 1,
    borderColor: colors.danger,
    borderRadius: 8,
    paddingHorizontal: 16,
    paddingVertical: 9,
  },
  cancelButtonText: {
    color: colors.danger,
    fontSize: 13,
    fontWeight: "600",
  },
  fullButton: {
    backgroundColor: colors.cardAlt,
    borderRadius: 8,
    paddingHorizontal: 16,
    paddingVertical: 9,
  },
  fullButtonText: {
    color: colors.muted,
    fontSize: 13,
    fontWeight: "600",
  },
  modalOverlay: {
    flex: 1,
    justifyContent: "flex-end",
    backgroundColor: "rgba(0,0,0,0.6)",
  },
  sheet: {
    backgroundColor: colors.card,
    borderTopLeftRadius: 18,
    borderTopRightRadius: 18,
    paddingHorizontal: 20,
    paddingTop: 20,
    paddingBottom: 24,
    maxHeight: "56%",
  },
  sheetHeader: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "flex-start",
    marginBottom: 10,
  },
  sheetTitle: {
    fontSize: 17,
    fontWeight: "700",
    color: colors.text,
  },
  sheetSubtitle: {
    fontSize: 13,
    color: colors.muted,
    marginTop: 2,
  },
  closeButton: {
    width: 28,
    height: 28,
    borderRadius: 14,
    backgroundColor: colors.cardAlt,
    alignItems: "center",
    justifyContent: "center",
  },
  closeButtonText: {
    fontSize: 14,
    color: colors.text,
  },
  rosterList: {
    flexGrow: 0,
  },
  rosterRow: {
    flexDirection: "row",
    alignItems: "center",
    gap: 10,
    paddingVertical: 8,
    borderBottomWidth: 1,
    borderBottomColor: colors.border,
  },
  avatar: {
    width: 32,
    height: 32,
    borderRadius: 16,
    backgroundColor: colors.cardAlt,
    alignItems: "center",
    justifyContent: "center",
  },
  avatarYou: {
    backgroundColor: colors.accent,
  },
  avatarText: {
    fontSize: 13,
    fontWeight: "700",
    color: colors.muted,
  },
  avatarTextYou: {
    color: colors.onAccent,
  },
  rosterName: {
    fontSize: 14,
    color: colors.text,
  },
  rosterNameYou: {
    fontWeight: "700",
    color: colors.accent,
  },
});
