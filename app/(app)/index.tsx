import CollapsibleSection from "@/components/CollapsibleSection";
import { colors } from "@/constants/theme";
import { useRouter } from "expo-router";
import {
  ScrollView,
  StyleSheet,
  Text,
  TouchableOpacity,
  View,
} from "react-native";

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

const TODAYS_WORKOUT = {
  title: "Full Body Strength",
  description:
    "Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.",
  exercises: [
    "3x10 Barbell Squat",
    "3x8 Bench Press",
    "3x10 Romanian Deadlift",
    "3x12 Dumbbell Row",
    "3x15 Overhead Press",
  ],
};

const days = [
  "Sunday",
  "Monday",
  "Tuesday",
  "Wednesday",
  "Thursday",
  "Friday",
  "Saturday",
];

export default function Home() {
  const router = useRouter();

  async function handleSignOut() {
    router.replace("/sign-in");
  }

  const date = new Date();

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      <View style={styles.header}>
        <Text style={styles.heading}>
          {`${days[date.getDay()]}, ${date.getMonth() + 1}/${date.getDate()}`}
        </Text>
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
        <View style={styles.card}>
          <Text style={styles.cardTitle}>{TODAYS_WORKOUT.title}</Text>
          <Text style={styles.cardBody}>{TODAYS_WORKOUT.description}</Text>
          <View style={styles.exerciseList}>
            {TODAYS_WORKOUT.exercises.map((exercise) => (
              <View key={exercise} style={styles.exerciseRow}>
                <View style={styles.bullet} />
                <Text style={styles.exerciseText}>{exercise}</Text>
              </View>
            ))}
          </View>
        </View>
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
    fontSize: 16,
    fontWeight: "600",
    color: colors.text,
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
