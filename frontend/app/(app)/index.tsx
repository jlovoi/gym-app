import CollapsibleSection from "@/components/CollapsibleSection";
import { colors } from "@/constants/theme";
import { useRouter } from "expo-router";
import { ScrollView, StyleSheet, Text, TouchableOpacity, View } from "react-native";
import { wodTypes, formatWodTitle, wodDescription } from "./utils/wods";

const ANNOUNCEMENTS = [
  {
    id: "1",
    title: "Holiday Hours",
    body: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
  },
  {
    id: "2",
    title: "New Equipment Arriving",
    body: "Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.",
  },
];

const TODAYS_WOD = [
  {
    title: null,
    type: wodTypes.EMOM,
    time: 10,
    interval: 2,
    description: null,
    exercises: ["EVEN MINUTE: 10 Push-ups", "ODD MINUTE: 15 Air Squats"],
  },
  {
    title: null,
    type: wodTypes.EMOM,
    time: 10,
    interval: 2,
    description: null,
    exercises: ["EVEN MINUTE: 10 Push-ups", "ODD MINUTE: 15 Air Squats"],
  },
];

const days = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

export default function Home() {
  const router = useRouter();

  async function handleSignOut() {
    router.replace("/sign-in");
  }

  const date = new Date();

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      <View style={styles.header}>
        <Text style={styles.heading}>{`${days[date.getDay()]}, ${date.getMonth() + 1}/${date.getDate()}`}</Text>
        <TouchableOpacity onPress={handleSignOut}>
          <Text style={styles.signOut}>Sign out</Text>
        </TouchableOpacity>
      </View>

      <CollapsibleSection title="Announcements">
        {ANNOUNCEMENTS.map((item) => (
          <View key={item.id} style={styles.card}>
            <Text style={styles.cardTitle}>{item.title}</Text>
            <Text style={styles.cardBody}>{item.body}</Text>
          </View>
        ))}
      </CollapsibleSection>

      <CollapsibleSection title="Today's Workout">
        {TODAYS_WOD.map((wod, index) => (
          <View style={styles.card} key={index}>
            <View style={styles.titleRow}>
              <View style={styles.indexBadge}>
                <Text style={styles.indexBadgeText}>{index + 1}</Text>
              </View>
              <Text style={styles.workoutTitle}>{wod.title || formatWodTitle(wod.type, wod.time, wod.interval)}</Text>
            </View>
            {wod.title && <Text style={styles.cardSubtitle}>{formatWodTitle(wod.type, wod.time, wod.interval)}</Text>}
            <Text style={styles.cardBody}>{wod.description || wodDescription[wod.type](wod.time, wod.interval)}</Text>
            <View style={styles.exerciseList}>
              {wod.exercises.map((exercise) => (
                <View key={exercise} style={styles.exerciseRow}>
                  <View style={styles.bullet} />
                  <Text style={styles.exerciseText}>{exercise}</Text>
                </View>
              ))}
            </View>
          </View>
        ))}
      </CollapsibleSection>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: colors.background,
  },
  content: {
    paddingHorizontal: 20,
    paddingTop: 60,
    paddingBottom: 40,
  },
  header: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
    marginBottom: 28,
  },
  heading: {
    fontSize: 28,
    fontWeight: "700",
    color: colors.text,
  },
  signOut: {
    color: colors.danger,
    fontSize: 14,
  },
  card: {
    backgroundColor: colors.card,
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
  },
  cardTitle: {
    fontSize: 20,
    fontWeight: "800",
    color: colors.text,
    marginBottom: 6,
  },
  titleRow: {
    flexDirection: "row",
    alignItems: "center",
    gap: 8,
    marginBottom: 6,
  },
  indexBadge: {
    width: 22,
    height: 22,
    borderRadius: 11,
    backgroundColor: colors.accent2,
    alignItems: "center",
    justifyContent: "center",
  },
  indexBadgeText: {
    fontSize: 12,
    fontWeight: "700",
    color: colors.background,
  },
  workoutTitle: {
    flexShrink: 1,
    fontSize: 20,
    fontWeight: "800",
    color: colors.text,
  },
  cardSubtitle: {
    fontSize: 14,
    fontWeight: "600",
    color: colors.muted,
    marginBottom: 6,
  },
  cardBody: {
    fontSize: 14,
    color: colors.muted,
    lineHeight: 20,
  },
  exerciseList: {
    marginTop: 12,
    gap: 8,
  },
  exerciseRow: {
    flexDirection: "row",
    alignItems: "center",
    gap: 8,
  },
  bullet: {
    width: 6,
    height: 6,
    borderRadius: 3,
    backgroundColor: colors.accent2,
  },
  exerciseText: {
    fontSize: 14,
    color: colors.text,
  },
});
