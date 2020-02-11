using Xunit;

namespace TestRunner
{
    public class GreetTests
    {
        [Fact]
        public void GreetNumber()
        {
            string result = IntegrationTests.GreetANumber(7);
            Assert.Equal("Hello, #7!", result);
        }

        [Fact]
        public void GreetNumberRepeated()
        {
            for (int number = 0; number < 1000; number += 1)
            {
                string actual = IntegrationTests.GreetANumber(number);
                Assert.Equal($"Hello, #{number}!", actual);
            }
        }

        [Fact]
        public void ReturnNumber()
        {
            int result = IntegrationTests.ReturnANumber();
            Assert.Equal(7, result);
        }

        [Fact]
        public void StringArg()
        {
            string result = IntegrationTests.StringArg("Test");
            Assert.Equal("Hello, Test!", result);
        }

        [Fact]
        public void StringArgRepeated()
        {
            for (int number = 0; number < 1000; number += 1)
            {
                string result = IntegrationTests.StringArg("Test");
                Assert.Equal("Hello, Test!", result);
            }
        }

        [Fact]
        public void CreatePersonInfo()
        {
            PersonInfo info = new PersonInfo("David", 12);
            Assert.Equal("David", info.Name());
            Assert.Equal(12, info.Age());
        }
    }
}